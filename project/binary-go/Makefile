.PHONY: clean

# Specify the target operating systems and architectures
TARGETS = \
    linux/386 \
    linux/amd64 \
    linux/arm \
    linux/arm64 \
    windows/386 \
    windows/amd64
	# no getDetailedPortsList() for darwin
    # darwin/amd64 \
    # darwin/arm64 \
	# random linux
    # linux/loong64 \
    # linux/mips \
    # linux/mipsle \
    # linux/mips64 \
    # linux/mips64le \
    # linux/ppc64 \
    # linux/ppc64le \
    # linux/riscv64 \
    # linux/s390x \

list:
	@echo $(TARGETS) | tr " " "\n"

# Set the default target to build all binaries
all: $(TARGETS)

# Define the build rule for each target
$(TARGETS):
	GOOS=$(word 1,$(subst /, ,$@)) GOARCH=$(word 2,$(subst /, ,$@)) go build -o bin/imposters-$(word 1,$(subst /, ,$@))-$(word 2,$(subst /, ,$@))

# Clean up the generated binaries
clean:
	rm -rf bin/*

run:
	go build .
	./cpe-426-project

watch:
	@echo main.go | entr -rc make run
