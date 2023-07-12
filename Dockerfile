FROM docker.io/ubuntu:20.04
ARG TARGETARCH

RUN mkdir -p /opt/libvirt_autoscaler
WORKDIR /opt/libvirt_autoscaler
COPY ./tools/target_arch.sh /opt/libvirt_autoscaler
COPY ./docker/config.toml /opt/libvirt_autoscaler/
RUN apt-get -y update \
  && DEBIAN_FRONTEND=noninteractive  apt-get -y install libvirt-dev
RUN --mount=type=bind,target=/context \
 cp /context/target/$(/opt/libvirt_autoscaler/target_arch.sh)/release/libvirt_autoscaler /opt/libvirt_autoscaler/libvirt_autoscaler
CMD ["/opt/libvirt_autoscaler/libvirt_autoscaler"]
EXPOSE 8443
