//go:build linux && !muslc && amd64 && !sys_novaproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lnovaproc.x86_64
import "C"
