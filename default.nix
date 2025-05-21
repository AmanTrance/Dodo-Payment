{ pkgs ? import <nixpkgs> { } } :
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ 
    zlib
    libgcc
    libgccjit
    libyaml
  ];

  buildInputs = with pkgs; [ 
    goose
    rustup
    docker
    containerd
  ];

  LD_LIBRARY_PATH = "${pkgs.zlib.outPath}/lib:${pkgs.libgcc.outPath}/lib";

  shellHook = ''
    rustup update stable
    rustup target add x86_64-unknown-linux-musl
    docker run --name=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres:latest
    docker run --name=rabbitmq -p 5672:5672 -d rabbitmq:latest
  '';
}
