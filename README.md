# Silly KVM

What if we could react to USB events and send monitor DDC commands? That's a KVM, Shirley?

## Using

```shell
Usage: silly-kvm [OPTIONS] --usb-vendor-id <USB_VENDOR_ID> --usb-product-id <USB_PRODUCT_ID>

Options:
  -d, --debug

  -v, --usb-vendor-id <USB_VENDOR_ID>
          USB Vendor ID to listen for
  -p, --usb-product-id <USB_PRODUCT_ID>
          USB Product ID to listen for
      --ddc-wait-interval <DDC_WAIT_INTERVAL>
          How long to pause after issuing a DDC command [default: 300]
  -m, --monitor-config <MONITOR_CONFIG>...
          Monitor configuration in the format <display_id>:<device_arrive_mode>:<device_left_mode>
  -h, --help
          Print help
```
