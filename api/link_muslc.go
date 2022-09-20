//go:build linux && muslc && !sys_novaproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lnovaproc_muslc
import "C"
