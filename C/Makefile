LINK := lusb-1.0
SOURCE := getInterface

make:
	gcc ${SOURCE}.c -o ${SOURCE} -${LINK}
        $(info Run executable with sudo)

makeUSB:
	arm-linux-gnueabihf-gcc -o ${SOURCE} ${SOURCE}.c -${LINK} -march=armv7-a -mfloat-abi=soft
