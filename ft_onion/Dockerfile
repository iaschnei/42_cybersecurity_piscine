FROM debian:latest

RUN  apt update && apt upgrade && apt install nginx

EXPOSE 80

COPY ./index.html   /website/index.html
COPY ./nginx.conf   /etc/nginx/nginx.conf
COPY ./sshd_config  /conf/sshd_config
COPY ./torrc        /conf/torrc
COPY ./init.sh      /init.sh