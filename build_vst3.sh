while getopts 'dr' OPTION; do
    case "$OPTION" in 
        d)
            echo "Building dev noisegen"
            cargo xtask bundle noisegen
            ;;
        r)
            echo "Building prod noisegen"
            cargo xtask bundle noisegen --release
            ;;
        ?)
            echo "No build version provided. Building prod noisegen"
            cargo xtask bundle noisegen --release
            ;;
    esac
done
shift "$(($OPTIND -1))"

echo "Removing old release version from VST3 folder."
rm -rf ~/Library/Audio/Plug-Ins/VST3/noisegen.vst3

echo "Moving new vst3 version into VST3 folder."
mv target/bundled/noisegen.vst3 ~/Library/Audio/Plug-Ins/VST3/noisegen.vst3
