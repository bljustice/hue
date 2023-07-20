while getopts 'dr' OPTION; do
    case "$OPTION" in 
        d)
            echo "Building dev hue"
            cargo xtask bundle hue
            ;;
        r)
            echo "Building prod hue"
            cargo xtask bundle hue --release
            ;;
        ?)
            echo "No build version provided. Building prod hue"
            cargo xtask bundle hue --release
            ;;
    esac
done
shift "$(($OPTIND -1))"

echo "Removing old release version from VST3 folder."
rm -rf ~/Library/Audio/Plug-Ins/VST3/hue.vst3

echo "Moving new vst3 version into VST3 folder."
mv target/bundled/hue.vst3 ~/Library/Audio/Plug-Ins/VST3/hue.vst3
