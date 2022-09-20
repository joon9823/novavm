//go:build darwin && !sys_novaproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lnovaproc
import "C"
