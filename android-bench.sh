
export PATH=$PATH:~/platform-tools/platform-tools/
export ANDROID_NDK_HOME=~/android-ndk-r27d-linux/android-ndk-r27d/

cargo dinghy -d android -p auto-android-aarch64 test
