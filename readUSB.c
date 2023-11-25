#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#include <libusb-1.0/libusb.h>

static int count = 0;

int hotplug_callback(struct libusb_context *ctx, struct libusb_device *dev,
			libusb_hotplug_event event, void *user_data) {
	static libusb_device_handle *dev_handle = NULL;
	struct libusb_device_descriptor desc;
	int rc;

	(void)libusb_get_device_descriptor(dev, &desc);

	if (LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED == event) {
		rc = libusb_open(dev, &dev_handle);
		if (LIBUSB_SUCCESS != rc){
			printf("Could not open USB device\n");
		}else{
			printf("Vendor:Device = %04x:%04x\n", desc.idVendor, desc.idProduct);
			count++;
		}
	} else if (LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT == event) {
	       if (dev_handle){
			libusb_close(dev_handle);
			dev_handle = NULL;
	       }
	} else {
		printf("Unhandled event %d\n", event);
	}
	//count++;

	return 0;	
}


int main(int argc, char *argv[]) {
	
	libusb_hotplug_callback_handle callback_handle;
	int rc;

	libusb_init(NULL);

	rc = libusb_hotplug_register_callback(NULL, LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED | 
			LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT, 0, LIBUSB_HOTPLUG_MATCH_ANY, 
			LIBUSB_HOTPLUG_MATCH_ANY, LIBUSB_HOTPLUG_MATCH_ANY, hotplug_callback, 
			NULL, &callback_handle);

	if (LIBUSB_SUCCESS != rc) {
		printf("Error creating a hotplug callback\n");
		libusb_exit(NULL);
		return EXIT_FAILURE;
	}
	
	struct libusb_context *context = NULL;
	struct libusb_device **list = NULL;
	rc = libusb_init(&context);
	if (LIBUSB_SUCCESS != rc) {
		printf("Error initializing context\n");
		libusb_exit(NULL);
		return EXIT_FAILURE;
	}

	while (count == 0/*count < 2*/){
		libusb_handle_events_completed(NULL, NULL);
		nanosleep(&(struct timespec){0, 10000000UL}, NULL);
	}
	libusb_get_device_list(context, &list);
	struct libusb_device *device = list[0];
	struct libusb_device_descriptor desc;
	rc = libusb_get_device_descriptor(device, &desc);
	if (LIBUSB_SUCCESS != rc){
		printf("Error retrieving descriptor\n");
	}
	//printf("%04x\n", LIBUSB_CLASS_PER_INTERFACE);
	printf("Vendor:Device:USBType:Class:Subclass:Protocol = %04x:%04x:%04x:%02x:%02x:%02x\n", desc.idVendor, desc.idProduct, desc.bcdUSB, desc.bDeviceClass, desc.bDeviceSubClass, desc.bDeviceProtocol);

	//struct libusb_interface_descriptor inter;
	//rc = libusb_get_descriptor(, LIBUSB_DT_INTERFACE, 


	libusb_hotplug_deregister_callback(NULL, callback_handle);	

	libusb_exit(NULL);

	return 0;
}
