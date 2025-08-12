# Entware Overlay

This page contains information on how to deploy an entware overlay on the Centauri Carbon. The carbon uses an armv7l architecture and a Linux 5.4 kernel (5.4.61-ab847), so we will be using the latest entware armv7sf-k3.2 build (for Linux kernel 3.2 and later).

This approcach leverages the `/user-resource` partition, which is 6.4GB from factory and formatted ext4, which is perfect for entware and the `/root` homedir, which should only take a few tens of MB!

## Entware Installation

1. Obtain a root shell on your Centauri Carbon.

1. Create the required directories and symlinks:
`mkdir -p /user-resource/entware /user-resource/root`
`ln -sf /user-resource/entware /opt`
`ln -sf /user-resource/root /root`

1. Set-up the new user root homedir to use entware bash as a shell and add entware to the PATH:
`echo 'exec /opt/bin/bash -l' > /root/.profile`
`echo 'export PATH=/opt/bin:/opt/sbin:$PATH' > /root/.bash_profile`

1. Run the entware installer
`wget -O - http://bin.entware.net/armv7sf-k3.2/installer/generic.sh | /bin/sh`

1. Temporarily add entware to your path, and install some useful packages!
`export PATH="/opt/sbin:/opt/bin:$PATH"`
`opkg update && opkg upgrade && opkg install dropbear bash strace ldd file gdb xxd`

1. Set root password to something you know
`passwd root`
`<enter new root password now>`

1. Start dropbear for this reboot
`/opt/etc/init.d/rc.unslung restart`

1. Make dropbear start persistent via /etc/rc.local
`sed -re 's|exit 0|# BEGIN: INITIALIZE ENTWARE\nsh -c "sleep 5 \&\& /opt/etc/init.d/rc.unslung start" \&\n# END: INITIALIZE ENTWARE\n\nexit 0|' -i /etc/rc.local`

1. SSH to your printer as root/<yourpassword> and profit!

### Installation Notes: 
- entware is installed to /opt which is a symlink to `/user-resource/entware`
- root homedir is symlinked to `/user-resource/root`
- /user-resource has ~6.4GB free from factory! It's where gcodes go
  - I tried to get this working w/ an ext2/3/4 partition on USB but it doesn't want to mount it!
  - mount seems funky in the distribution and seems to only like fat partitions?
- scp works but dropbear does not support sftp
- This should be persistent across reboot, although its unknown how a firmware update would affect this. If you try it then please report back!

## Entware Uninstall
This reverts to stock Centauri Carbon, uninstalls entware and delete all edits we made.

1. Obtain a root shell on your Centauri Carbon.

1. Stop running entware services
`/opt/etc/init.d/rc.unslung stop`

1. Clean-up symlinks
`rm -f /opt /root`

1. Delete edits we made to overlay (this reverts files back to stock from RO flash)
`cd /overlay/upper/etc`
`rm -f rc.local passwd* shadow*`

1. (Optional) Clean-up user folders from the /user-resource partition. Note this will delete all of your installed packages and files in /root homedir!
`cd /user-resource`
`rm -Rf root/ entware/`

1. Reboot your printer
`reboot`
