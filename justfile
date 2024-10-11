build:
	cargo build --release

install:
	cargo install --path .
	cp silly-kvm.service ~/.config/systemd/user/silly-kvm.service
	systemctl --user daemon-reload
	systemctl restart --user silly-kvm
