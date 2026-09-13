EXE = WakWak

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
	cargo rustc --release -- --emit link=$(NAME)
