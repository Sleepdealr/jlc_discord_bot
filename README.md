# Jlc Discord Bot
Discord bot to check JLC's component stock
Can be adapted for other websites

Built with Rust and Postgres

Discord permalink: https://discord.gg/8D5hJeY8AB

# Usage
This will be for users with systemd, if you are using another init system, you will need to modify these files
1. Install cargo
2. Fill out the `start.sh` and `jlc.service` files with the necessary information
   1. Move `start.sh` to main directory
   2. Move `jlc.service` to services directory, usually `/lib/systemd/service/`
3. Run `chmod +x start.sh`
4. Create a database postgres database and put the URL in `.env`. Fill out the rest of `.env`
    1. Contact me for help with setup, I'll make a first time setup later
6. Allow `start.sh` to be executed by running `chmod +x start.sh`
7. Enable the service with `sudo systemctl enable jlc`
   