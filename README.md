# Jlc Discord Bot
Discord bot to check JLC's component stock
Can be adapted for other websites

Built with Rust and Postgres

# Running as Daemon
This will be for users with systemd, if you are using another init system, you will need to modify these files
1. Install cargo
2. Fill out the `start.sh` and `jlc.service` files with the necessary information, move to main directory
3. Run `chmod +x start.sh`
4. Create a database postgres database and put the URL in `.env`. Fill out the rest of `.env`
   1. Contact me for help with setup, I'll make a first time setup later
5. Move `jlc.service` to `/lib/systemd/service/`
6. Allow `start.sh` to be executed by running `chmod +x start.sh`
7. Enable the service with `sudo systemctl enable jlc`
   1. If you're doing this for the first time, run `sudo systemctl start jlc` to start it
   