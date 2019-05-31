#!/usr/bin/env bash

for dir in /home/vcap/deps/0/apt/usr/lib/R/site-library/*; do
    if [[ ! "$dir" =~ (htmlwidgets|shiny|httpuv) ]]; then
       echo installing $dir;
       R CMD INSTALL $dir;
    fi
done

/home/vcap/deps/0/apt/usr/sbin/nginx -V
/home/vcap/deps/0/apt/usr/sbin/nginx -p . -c nginx.conf &
R -f code.R
