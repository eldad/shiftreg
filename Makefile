NAME := shiftreg

.PHONY: deploy
deploy: src/$(NAME).icon
	cargo build
	flipperCmd PUT ./target/thumbv7em-none-eabihf/debug/$(NAME).fap /ext/apps/Dev

.PHONY: icon
icon: src/$(NAME).icon

src/$(NAME).icon: src/$(NAME).png
	./mkicon.sh $(NAME)
