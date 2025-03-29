FROM node:alpine

RUN npm install -g postgraphile

EXPOSE 5000
ENTRYPOINT ["postgraphile", "-n", "0.0.0.0"]