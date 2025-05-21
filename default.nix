{ pkgs ? import <nixpkgs> { } } :
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ 
    zlib
  ];

  buildInputs = with pkgs; [ 
    goose
  ];

  LD_LIBRARY_PATH = "${pkgs.zlib.outPath}/lib";
}
