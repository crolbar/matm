{
  rustPlatform,
  pkg-config,
  openssl,
  perl,
}:
rustPlatform.buildRustPackage {
  pname = "matm";
  version = "0.1";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [pkg-config perl];

  buildInputs = [openssl];
}
