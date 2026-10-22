#!/bin/bash
# Copies ffmpeg, ffprobe and whisper-cli with every non-system library into an app bundle,
# so GnaegiCut runs on Macs without Homebrew. Usage: bundle-tools.sh path/to/GnaegiCut.app
set -e
APP=$1
BIN="$APP/Contents/MacOS"
LIB="$APP/Contents/Frameworks"
BREW=$(brew --prefix)
GGML=$(brew --prefix ggml)
mkdir -p "$BIN" "$LIB"

# Finds the real file behind a load command; @rpath and @loader_path are resolved like dyld would.
resolve() { # dep referrer
  case $1 in
    @rpath/* | @loader_path/*)
      name=${1##*/}
      dir=$(dirname "$(readlink -f "$2")")
      for d in "$dir" "$dir/../lib" "$GGML/lib" "$BREW/lib"; do
        [ -f "$d/$name" ] && { readlink -f "$d/$name"; return; }
      done
      echo "cannot resolve $1 for $2" >&2 && exit 1 ;;
    *) readlink -f "$1" ;;
  esac
}

# Rewrites every Homebrew reference of one file to the copy in Frameworks, queueing new libraries.
relink() { # file original
  otool -L "$1" | tail -n +2 | awk '{print $1}' | while read -r dep; do
    case $dep in /usr/lib/* | /System/*) continue ;; esac
    name=${dep##*/}
    [ "$name" = "${1##*/}" ] && continue
    if [ ! -f "$LIB/$name" ]; then
      cp "$(resolve "$dep" "$2")" "$LIB/$name"
      chmod u+w "$LIB/$name"
      echo "$LIB/$name $(resolve "$dep" "$2")" >>"$QUEUE"
    fi
    install_name_tool -change "$dep" "@executable_path/../Frameworks/$name" "$1" 2>/dev/null
  done
  case $1 in *.dylib) install_name_tool -id "@executable_path/../Frameworks/${1##*/}" "$1" 2>/dev/null ;; esac
}

QUEUE=$(mktemp)
# ggml loads its Metal and CPU backends at runtime from the executable's folder.
for src in "$(brew --prefix ffmpeg-full)/bin/ffmpeg" "$(brew --prefix ffmpeg-full)/bin/ffprobe" \
  "$BREW/bin/whisper-cli" "$GGML"/libexec/*.so; do
  cp "$(readlink -f "$src")" "$BIN/${src##*/}"
  chmod u+w "$BIN/${src##*/}"
  echo "$BIN/${src##*/} $(readlink -f "$src")" >>"$QUEUE"
done
done_lines=0
while [ "$done_lines" -lt "$(wc -l <"$QUEUE")" ]; do
  done_lines=$((done_lines + 1))
  line=$(sed -n "${done_lines}p" "$QUEUE")
  relink "${line%% *}" "${line#* }"
done
rm "$QUEUE"

# ggml checks its Homebrew backend folder before the executable's; point it at an empty folder
# (same length, padded with slashes) so the bundled backends always win.
LIBEXEC=$(readlink -f "$GGML/libexec")
EMPTY=$(printf '/var/empty%*s' $((${#LIBEXEC} - 10)) '' | tr ' ' '/')
perl -pi -e "s|\Q$LIBEXEC\E|$EMPTY|g" "$LIB/libggml.0.dylib"

# Homebrew rpaths would make dyld look outside the bundle; drop them.
for f in "$BIN"/* "$LIB"/*; do
  otool -l "$f" | awk '/LC_RPATH/{getline; getline; print $2}' | while read -r rp; do
    install_name_tool -delete_rpath "$rp" "$f" 2>/dev/null || true
  done
  codesign --force --sign - "$f" 2>/dev/null
done
echo "Bundled tools: $(ls "$LIB" | wc -l | tr -d ' ') libraries, $(du -sh "$LIB" | cut -f1)"
