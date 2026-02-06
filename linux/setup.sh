#!/bin/bash

printf "DigiSafe setup started.\n"

mkdir -p ~/.config/digisafe
chmod 700 ~/.config/digisafe

systemd-ask-password -n "Enter pepper seed:" | \
	argon2 digipepper -id -t 4 -m 22 -p 1 -l 32 -r -v 13 | \
	systemd-creds encrypt --user --name=digipepper - ~/.config/digisafe/digipepper.cred

printf "Provisioning Backblaze API Bundle...\n"
{
	systemd-ask-password -n "Enter API Key Name:"
	printf "\0"
	systemd-ask-password -n "Enter API Key ID:"
	printf "\0"
	systemd-ask-password -n "Enter App Secret Key:"
} | systemd-creds encrypt --user --name=backblaze - ~/.config/digisafe/backblaze.cred

chmod 600 ~/.config/digisafe/*

cargo deb --install

printf "DigiSafe setup finished.\n"
