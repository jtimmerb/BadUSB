#include <stdio.h>
#include <libusb-1.0/libusb.h>

#define AUDIO 0x01
#define COMM 0x02
#define HID 0x03
#define PID 0x05
#define IMAGE 0x06
#define PRINTER 0x07
#define STOR 0x08
#define HUB 0x09
#define CDC 0x0A
#define SMART 0x0B
#define SECUR 0x0D
#define VIDEO 0x0E
#define HEALTH 0x0F

#define HID_KEYBOARD 0x01
#define HID_MOUSE 0x02

void print_interface(const struct libusb_interface_descriptor*);
void print_endpoint(const struct libusb_endpoint_descriptor*);
void read_from_usb(struct libusb_device_handle* , uint8_t, uint16_t); 

void list_usb_interfaces() {
    libusb_context* context = NULL;
    libusb_init(&context);
    int rc;
    libusb_device** devices;
    ssize_t count = libusb_get_device_list(context, &devices);
    if (count < 0) {
        fprintf(stderr, "Failed to get device list\n");
        libusb_exit(context);
        return;
    }

    printf("List of USB interfaces:\n");

    for (ssize_t i = 0; i < count; ++i) {
        libusb_device* device = devices[i];

        struct libusb_device_descriptor desc;
        libusb_get_device_descriptor(device, &desc);

        printf("Device %04x:%04x\n", desc.idVendor, desc.idProduct);
	struct libusb_device_handle* dev_handle;
        rc = libusb_open(device, &dev_handle);
	if (rc < 0){
		printf("Error Opening Device\n");
	}
	for (uint8_t j = 0; j < desc.bNumConfigurations; ++j) {
            struct libusb_config_descriptor* config;
            libusb_get_config_descriptor(device, j, &config);

            for (int k = 0; k < config->bNumInterfaces; ++k) {
                const struct libusb_interface* iface = &config->interface[k];
		const struct libusb_interface_descriptor* ifacedesc = &iface->altsetting[0];
		print_interface(ifacedesc);
		for (uint8_t l = 0; l < ifacedesc->bNumEndpoints; ++l){
			const struct libusb_endpoint_descriptor* endpdesc = &ifacedesc->endpoint[l];
			print_endpoint(endpdesc);
			read_from_usb(dev_handle, endpdesc->bEndpointAddress, endpdesc->wMaxPacketSize);
		}
            }

            libusb_free_config_descriptor(config);
        }
    }

    libusb_free_device_list(devices, 1);
    libusb_exit(context);
}

void print_interface(const struct libusb_interface_descriptor* ifacedesc) {
	int class = ifacedesc->bInterfaceClass;
	int sclass = ifacedesc->bInterfaceSubClass;
	int prot = ifacedesc->bInterfaceProtocol;
	char* class_str;
	char* prot_str;	
		
	switch(class) {
		case AUDIO:
			class_str = "Audio";
			prot_str = "Don't Care";
			break;
		case COMM:
			class_str = "Communications";
			prot_str = "Don't Care";
			break;
		case HID:
			class_str = "Human Interface Device";
			switch(prot) {
				case HID_KEYBOARD:
					prot_str = "Keyboard";
					break;
				case HID_MOUSE:
					prot_str = "Mouse";
					break;
				default: 
					prot_str = "Don't Care";
					break;
			}
			break;
		default:
			class_str = "Don't Care";	
			prot_str = "Don't Care"; 	
	}
	printf("  Interface %d\n", ifacedesc->bInterfaceNumber);	
	printf("   Class: %d %s\n", class, class_str);	
	printf("   Subclass: %d\n", sclass);
	printf("   Protocol: %d %s\n", ifacedesc->bInterfaceProtocol, prot_str);
}
        
void print_endpoint(const struct libusb_endpoint_descriptor* endpdesc) {
	uint8_t addr = endpdesc->bEndpointAddress;
	uint16_t maxPacketSize = endpdesc->wMaxPacketSize;
	printf("	EndPoint Address: %d\n", addr);
	printf("	 Max Packet Length: %d\n", maxPacketSize);
}

void read_from_usb(struct libusb_device_handle* handle, uint8_t addr, uint16_t maxPacketSize) {
    unsigned char buffer[maxPacketSize];
    int transferred;

    int result = libusb_bulk_transfer(handle, addr, buffer, sizeof(buffer), &transferred, 1000);

    if (result == 0) {
        // Successfully read data
        printf("Read %d bytes from USB: ", transferred);
        for (int i = 0; i < transferred; ++i) {
            printf("%02X ", buffer[i]);
        }
        printf("\n");
    } else {
        fprintf(stderr, "Error reading from USB: %s\n", libusb_strerror(result));
    }
}

int main() {
    list_usb_interfaces();
    return 0;
}

