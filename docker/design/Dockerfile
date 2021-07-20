
FROM ubuntu:20.04

# set noninteractive installation
ARG DEBIAN_FRONTEND=noninteractive

#install software requirements
RUN apt update && apt -y install pandoc \
    texlive-latex-recommended \
    texlive-fonts-recommended \
    texlive-latex-extra

#Add entry script
COPY docker-entrypoint.sh /
ENTRYPOINT [ "/docker-entrypoint.sh" ]
