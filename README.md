wait
====

A tiny utility container that can be used to block and wait for other
container's TCP sockets to become available before exiting.

This is intented to be used as part of CI pipelines, or other services that
need to wait for the dependent services to become available before starting.

This contains a single binary that can be copied into another container for
when its not possible to chain multiple containers together, like in
Kubernetes.

Usage - Docker
====

In docker/docker-compose, the container will discover any exposed ports of
linked containers and will block until the ports are available.

```
$ docker run -d --name nginx nginx
$ docker run --link nginx andyfoston/wait
Connecting to 172.17.0.2:80  Connected!
All targets are up!
```

Ports to poll can also be manually specified using the `WAIT_TARGETS` or
`TARGETS` environment variables: 

```
$ docker run -e TARGETS=www.google.com:443,hub.docker.com:443 andyfoston/wait
Connecting to www.google.com:443  Connected!
Connecting to hub.docker.com:443  Connected!
All targets are up!
```

Other Usage
====

The `wait` file is a self-contained binary and can be copied into other
containers, to wait for components to become available before starting another
process. This might be useful in Kubernetes maybe as an alternative to using
and init container to wait for a service to start.

(untested example):

```
FROM python:3.7

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY requirements.txt ./
RUN pip install -r requirements.txt
COPY . .
COPY --from=andyfoston/wait ./wait .
ENV WAIT_TARGETS=db:3306

CMD ["sh", "-c", "./wait && python manage.py migrate"]
```
