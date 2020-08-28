package wrapper

import (
	"os"
)

// FileExists validate if a given path is point to a valid file
func FileExists(file string) bool {
	info, err := os.Stat(file)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}