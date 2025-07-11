# PasswordManager-Backend
That's the backend you should use for self-hosting. Not a piece of art, but usable, and you can fork it anytime and improve it.
It consists of 2 big main juicy fat modules: users part (auth, user info and etc) and storage part (uploading and downloading).
Techonologies used:
- Rust
- MinIO Object Storage
- MariaDB database (or MySql, since MariaDB and MySql are interchangable (i think, again, I'm not double-checking that))

### Installation

1. Clone the repo
```bash
git clone https://github.com/commmrade/PasswordManager-Backend
```
2. Build the image
```bash
docker build . --tag passwordmanager-backend
```
Then you need to copy the image to your server any way you like
3. Copy server-compose... .yaml file to your server and compose it up, also copy pm.sql
4. Load the `pm.sql` dump to the database
```bash
docker cp pm.sql mariadb:/pm.sql # copy dump to the container filesystem
docker exec -it mariadb sh -c 'mariadb -u root -proot pm < /dump.sql' # load the dump
```
5. Create a `user-storages` bucket
6. At this point, `password-manager-backend` may be down, so start it again
```bash
docker start password-manager-backend
```
7. Should work at this point