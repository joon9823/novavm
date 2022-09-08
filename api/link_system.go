//go:build sys_kernelproc

package api

// #cgo LDFLAGS: -lkernelproc
import "C"
