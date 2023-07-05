pub(crate) const VM_XML: &str = r###"
<domain type="kvm">
  <name>HOSTNAME</name>
  <metadata>
    <libosinfo:libosinfo xmlns:libosinfo="http://libosinfo.org/xmlns/libvirt/domain/1.0">
      <libosinfo:os id="http://ubuntu.com/ubuntu/22.04"/>
    </libosinfo:libosinfo>
  </metadata>
  <memory>8388608</memory>
  <currentMemory>8388608</currentMemory>
  <vcpu>8</vcpu>
  <sysinfo type="smbios">
    <system>
      <entry name="serial">ds=nocloud-net;s=http://10.174.5.25:8080/;h=HOSTNAME</entry>
    </system>
  </sysinfo>
  <os>
    <type arch="x86_64" machine="q35">hvm</type>
    <boot dev="hd"/>
    <smbios mode="sysinfo"/>
  </os>
  <features>
    <acpi/>
    <apic/>
  </features>
  <cpu mode="host-passthrough"/>
  <clock offset="utc">
    <timer name="rtc" tickpolicy="catchup"/>
    <timer name="pit" tickpolicy="delay"/>
    <timer name="hpet" present="no"/>
  </clock>
  <pm>
    <suspend-to-mem enabled="no"/>
    <suspend-to-disk enabled="no"/>
  </pm>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type="file" device="disk">
      <driver name="qemu" type="qcow2" cache="none"/>
      <source file="/var/lib/libvirt/images/HOSTNAME.qcow2"/>
      <target dev="vda" bus="virtio"/>
    </disk>
    <controller type="usb" model="qemu-xhci" ports="15"/>
    <controller type="pci" model="pcie-root"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <interface type="network">
      <source network="NETWORK-NAME"/>
      <model type="virtio"/>
    </interface>
    <console type="pty"/>
    <channel type="unix">
      <source mode="bind"/>
      <target type="virtio" name="org.qemu.guest_agent.0"/>
    </channel>
    <input type="tablet" bus="usb"/>
    <graphics type="vnc" port="-1" listen="0.0.0.0"/>
    <video>
      <model type="vga"/>
    </video>
    <memballoon model="virtio"/>
    <rng model="virtio">
      <backend model="random">/dev/urandom</backend>
    </rng>
  </devices>
</domain>
"###;

pub(crate) const VOL_XML: &str = r###"
    <volume type='file'>
        <name>HOSTNAME.qcow2</name>
        <key>/var/lib/libvirt/images/HOSTNAME.qcow2</key>
        <target>
        <path>/var/lib/libvirt/images/HOSTNAME.qcow2</path>
        <format type='qcow2'/>
        </target>
      <backingStore>
        <path>/var/lib/libvirt/images/base/jammy-server-cloudimg-amd64.img</path>
        <format type='qcow2'/>
        </backingStore>
    </volume>
"###;
