EXE = WakWak

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
PYTHON ?= py -3
else
NAME := $(EXE)
PYTHON ?= python3
endif

native:
ifndef EVALFILE
	$(PYTHON) ./download_net.py
endif
	cargo rustc --release -- --emit link=$(NAME)
