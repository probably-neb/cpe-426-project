.PHONY: build, run, run-bash
build:
	sudo docker buildx build . -t chipwhisperer

run:
	sudo docker run -it -p 8888:8888 --privileged -v /dev/bus/usb:/dev/bus/usb -v ./chipwhisperer/:/home/imposters/chipwhisperer chipwhisperer jupyter notebook --ip 0.0.0.0 --port 8888 --allow-root --no-browser

run-bash:
	sudo docker run -it -p 8888:8888 --privileged -v /dev/bus/usb:/dev/bus/usb chipwhisperer
