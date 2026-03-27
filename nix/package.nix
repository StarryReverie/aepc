{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "aepc";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    description = "Arknights: Endfield Pipeline Calculator in TUI";
    homepage = "https://github.com/starryreverie/aepc";
    mainProgram = "aepc";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ starryreverie ];
    platforms = lib.platforms.unix;
  };
}
