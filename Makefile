EXE = WakWak

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
ifndef EVALFILE
	python3 ./download_net.py
endif
	cargo rustc --release -- --emit link=$(NAME)
