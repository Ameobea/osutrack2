if [ ! $(which cargo) ]; then
  echo "Osu!track requires that you have an active Rust installation in order to build.";
  echo "Check out https://rustup.rs for a super easy rust installer and verison manager.";
  exit 1;
fi

cd backend && cargo build
