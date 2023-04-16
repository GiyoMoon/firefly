# Creating a bootable sd card image for the Mango Pi MQ Quad
## Setup
```shell
apt update
apt install -y git vim make gcc pkg-config zlib1g-dev libusb-1.0-0-dev libfdt-dev libncurses-dev bison flex python3-setuptools swig python3-dev libssl-dev bc kmod rsync u-boot-tools gcc-aarch64-linux-gnu file wget cpio unzip fdisk

cd /root
mkdir project
cd project

git clone --depth 1 https://github.com/linux-sunxi/sunxi-tools
cd sunxi-tools
make
cd ..
```

## Build u-boot
```shell
git clone  --depth 1 https://github.com/ARM-software/arm-trusted-firmware.git
cd arm-trusted-firmware
make CROSS_COMPILE=aarch64-linux-gnu- PLAT=sun50i_h616 DEBUG=1 bl31
cd ..

git clone  --depth 1 git://git.denx.de/u-boot.git
cd u-boot
```
edit `drivers/power/axp305.c` and insert [this content](./config/axp305.c).
```
make CROSS_COMPILE=aarch64-linux-gnu- BL31=../arm-trusted-firmware/build/sun50i_h616/debug/bl31.bin orangepi_zero2_defconfig
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- menuconfig
```
Disable `[ ] Networking support` and save config as `.config`.
```
make CROSS_COMPILE=aarch64-linux-gnu- BL31=../arm-trusted-firmware/build/sun50i_h616/debug/bl31.bin
cd ..
```

### (Optional) test u-boot on Mango Pi
To test u-boot, make sure the Mango Pi is plugged in over usb (use the most right usb port on the Pi), and make sure NO sdcard is plugged in.
```shell
./sunxi-tools/sunxi-fel -v uboot ./u-boot/u-boot-sunxi-with-spl.bin
```

## Build linux kernel
At the time of writing this, stable was `6.2.11`.
```shell
git clone --depth 1 https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git
cd linux
```
Support for the Allwinner H616 in mainline Linux is fairly new and not all drives have been added yet. The Wifi drivers is missing for example. Here's how we can add it:
```shell
git clone --depth 1 https://github.com/lwfinger/rtl8723ds drivers/net/wireless/realtek/rtl8723ds
```
Modify `drivers/net/wireless/realtek/rtl8723ds/Kconfig` and delete the hypens around `--help--`.

Edit the `drivers/net/wireless/realtek/Kconfig` file and insert `source "drivers/net/wireless/realtek/rtl8723ds/Kconfig"`.

Edit the `drivers/net/wireless/realtek/Makefile` file and insert `obj-$(CONFIG_RTL8723DS)		+= rtl8723ds/`.

edit `arch/arm64/boot/dts/allwinner/sun50i-h616-orangepi-zero2.dts` and insert [this content](./config/sun50i-h616-orangepi-zero2.dts).
```shell
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- defconfig
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- menuconfig
```
Enable the `Realtek 8723D SDIO or SPI WiFi` module under `Device Drivers` -> `Network device support` -> `Wireless LAN` -> `Realtek devices`.
```shell
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j8 Image
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j8 dtbs

make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j8 modules
cd ..
```

## Creating a root file system with Alpine
```shell
git clone --depth 1 https://github.com/alpinelinux/alpine-make-rootfs.git
cd alpine-make-rootfs/
./alpine-make-rootfs --branch v3.17 --packages 'wpa_supplicant' rootfs.tar
cd ..
```

## Creating a bootable image
Note that this step won't work in a docker container, because loop devices don't exist there.

```shell
mkdir image
cd image

cp ../u-boot/u-boot-sunxi-with-spl.bin .
cp ../linux/arch/arm64/boot/Image .
cp ../linux/arch/arm64/boot/dts/allwinner/sun50i-h616-orangepi-zero2.dtb .
cp ../alpine-make-rootfs/rootfs.tar .

dd if=/dev/zero of=./alpine.img bs=1M count=512
```
Create two partitions (boot and root) with `fdisk`:
```shell
fdisk ./alpine.img
n p 1 40960 +131072
n p 2 172033 default
w
```
Setup up loop devices and mount them. You can always check them with `losetup -l`:
```shell
losetup -f ./alpine.img

dd if=./u-boot-sunxi-with-spl.bin of=/dev/{loop0} bs=8K seek=1

# -o: starting sector * sector size, --sizelimit: number of sectors * sector size
# -o: 40960 * 512, --sizelimit: 131072 * 512
losetup -f -o 20971520 --sizelimit 67109376 alpine.img
mkfs.fat /dev/{loop1}
# -o: 172033 * 512, --sizelimit: ? * 512
losetup -f -o 88080896 --sizelimit 448790016 alpine.img
mkfs.ext4 /dev/{loop2}

mkdir /mnt/boot/
mkdir /mnt/rootfs/
mount /dev/loop1 /mnt/boot/
mount /dev/loop2 /mnt/rootfs/
```
Create a `boot.cmd` file:
```cmd
setenv bootargs console=ttyS0,115200 root=/dev/mmcblk0p2 rootfstype=ext4 rootwait rw
setenv bootcmd fatload mmc 0:1 0x4fc00000 boot.scr; fatload mmc 0:1 0x40200000 Image; fatload mmc 0:1 0x4fa00000 sun50i-h616-orangepi-zero2.dtb; booti 0x40200000 - 0x4fa00000
```
```shell
mkimage -C none -A arm64 -T script -d boot.cmd boot.scr
cp ./Image /mnt/boot/
cp ./sun50i-h616-orangepi-zero2.dtb /mnt/boot/
cp ./boot.scr /mnt/boot/
tar -vxf ./rootfs.tar -C /mnt/rootfs/

cd ../linux
make INSTALL_MOD_PATH=/mnt/rootfs/ modules_install
cd ../image

sync
umount /mnt/rootfs
umount /mnt/boot

losetup -d /dev/loop1
losetup -d /dev/loop2
losetup -d /dev/loop0
```

## Write image to sd card
```shell
dd if=./alpine.img of=/dev/sdd
```
