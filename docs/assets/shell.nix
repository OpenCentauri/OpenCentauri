{
  pkgs ? import (
    let
      tryNixpkgs = builtins.tryEval <nixpkgs>;
    in
    if tryNixpkgs.success then
      tryNixpkgs.value
    else
      fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz"
  ) { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    (python3.withPackages (ps: with ps; [
      pycryptodome
      requests
      paramiko
      scp
      click
      rich
    ]))

    squashfsTools
    cpio
    unzip
    p7zip
    binwalk
    file
    hexdump
    radare2
    socat
    netcat
    tcpdump
    mosquitto
    openssh
    openssl
    jq
    yq
    tree
    git
    vim
    curl
    wget
  ];

  shellHook = ''
    FW_FILE="810e5a7e9518452c9172e11a7d04a683.bin"
    KEY=
    IV=

    [ ! -f unpack.py ] && wget -q http://suchmememanyskill.github.io/OpenCentauri/assets/unpack.py
    [ ! -f "$FW_FILE" ] && wget -q https://download.chitubox.com/chitusystems/chitusystems/public/printer/firmware/release/1/ca8e1d9a20974a5896f8f744e780a8a7/1/1.1.29/2025-06-18/"$FW_FILE"

    if [ ! -f update.swu ]; then
      python unpack.py "$FW_FILE" "$KEY" "$IV"
    fi

    if [ ! -d extracted ]; then
      mkdir -p extracted
      cpio -idv -D extracted < update.swu
    fi

    if [ ! -d squashfs-root ]; then
      unsquashfs extracted/rootfs
    fi
  '';
}
