cargo xtask bundle noisegen --release 

echo "Removing old release version from VST3 folder."
rm -rf ~/Library/Audio/Plug-Ins/VST3/noisegen.vst3

echo "Moving new vst3 version into VST3 folder."
mv target/bundled/noisegen.vst3 ~/Library/Audio/Plug-Ins/VST3/noisegen.vst3
