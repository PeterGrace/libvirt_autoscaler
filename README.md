# libvirt_autoscaler

This project implements a gRPC server based on the [externalgrpc Cloud Provider for the Kubernetes autoscaler project.](https://github.com/kubernetes/autoscaler/tree/master/cluster-autoscaler/cloudprovider/externalgrpc)

What this server does is communicate to a libvirt/kvm installation to enable libvirt/kvm as a target for cluster autoscaling.

This code assumes that on the libvirt side, you already have a way of dynamically provisioning a node (I use cloud-init and will include an example userdata.)

This was mostly a pet project for myself but I welcome submissions from others and/or assistance in productionizing it further if this appeals to your use cases.
