//go:build linux && !muslc && arm64 && !sys_novaproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lnovaproc.aarch64
import "C"
