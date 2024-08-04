package main

import (
	"bufio"
	"bytes"
	"flag"
	"fmt"
	"io"
	"math/rand"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"time"
)

const (
	chunkSize = 1024
)

var (
	input       string
	mode        string
	countFrames int
	positFrames int
	audio       bool
	firstFrame  bool
	kill        float64
)

func init() {
	flag.StringVar(&input, "i", "", "input file")
	flag.StringVar(&mode, "m", "void", "choose mode - void, random, reverse, invert, bloom, pulse, jiggle, overlap")
	flag.IntVar(&countFrames, "c", 1, "how often to glitch (for modes that support it)")
	flag.IntVar(&positFrames, "n", 1, "how many frames in the glitch (for modes that support it)")
	flag.BoolVar(&audio, "a", false, "attempt to preserve audio")
	flag.BoolVar(&firstFrame, "ff", true, "whether to keep first video frame")
	flag.Float64Var(&kill, "k", 0.7, "max framesize to kill while cleaning")
}

func main() {
	timerStart := time.Now()

	printBanner()

	flag.Parse()

	if input == "" || !fileExists(input) {
		fmt.Println("> step 0/5: valid input file required!")
		fmt.Println("use -h to see help")
		os.Exit(1)
	}

	tempDir := fmt.Sprintf("temp-%d", rand.Intn(89999)+10000)
	tempHdrl := filepath.Join(tempDir, "hdrl.bin")
	tempMovi := filepath.Join(tempDir, "movi.bin")
	tempIdx1 := filepath.Join(tempDir, "idx1.bin")

	os.Mkdir(tempDir, 0755)
	defer os.RemoveAll(tempDir)

	fmt.Println("> step 1/5 : streaming into binary files")

	moviMarkerPos := bstreamUntilMarker(input, tempHdrl, []byte("movi"), 0)
	idx1MarkerPos := bstreamUntilMarker(input, tempMovi, []byte("idx1"), moviMarkerPos)
	bstreamUntilMarker(input, tempIdx1, nil, idx1MarkerPos)

	fmt.Println("> step 2/5 : constructing frame index")

	frameTable := buildFrameTable(tempMovi)
	clean := cleanFrames(frameTable)

	fmt.Printf("> step 3/5 : mode %s\n", mode)

	final := applyEffect(clean)

	fmt.Println("> step 4/5 : putting things back together")

	cname := ""
	if countFrames > 1 {
		cname = fmt.Sprintf("-c%d", countFrames)
	}
	pname := ""
	if positFrames > 1 {
		pname = fmt.Sprintf("-n%d", positFrames)
	}
	fileout := fmt.Sprintf("%s-%s%s%s.avi", input[:len(input)-4], mode, cname, pname)

	if fileExists(fileout) {
		os.Remove(fileout)
	}

	bstreamUntilMarker(tempHdrl, fileout, nil, 0)
	writeFrames(tempMovi, fileout, final)
	bstreamUntilMarker(tempIdx1, fileout, nil, 0)

	fmt.Printf("> step 5/5 : done - final idx size : %d\n", len(final))

	timerEnd := time.Now()
	// print elapsed ms
	fmt.Printf("elapsed time: %v\n", timerEnd.Sub(timerStart))
}

func printBanner() {
	fmt.Println(" _                        _        ")
	fmt.Println("| |                      | |       ")
	fmt.Println("| |_ ___  _ __ ___   __ _| |_ ___  ")
	fmt.Println("| __/ _ \\| '_ ` _ \\ / _` | __/ _ \\ ")
	fmt.Println("| || (_) | | | | | | (_| | || (_) |")
	fmt.Println(" \\__\\___/|_| |_| |_|\\__,_|\\__\\___/ ")
	fmt.Println("tomato.go v1.0 last update " + time.Now().Format("02.01.2006"))
	fmt.Println("\\\\ Audio Video Interleave breaker")
	fmt.Println(" ")
	fmt.Println("glitch tool made with love for the glitch art community <3")
	fmt.Println("if you have any questions, would like to contact me")
	fmt.Println("or even hire me for performance / research / education")
	fmt.Println("you can shoot me an email at kaspar.ravel@gmail.com")
	fmt.Println("___________________________________")
	fmt.Println(" ")
	fmt.Println("wb. https://www.kaspar.wtf ")
	fmt.Println("fb. https://www.facebook.com/kaspar.wtf ")
	fmt.Println("ig. https://www.instagram.com/kaspar.wtf ")
	fmt.Println("___________________________________")
	fmt.Println(" ")
}

func fileExists(filename string) bool {
	_, err := os.Stat(filename)
	return !os.IsNotExist(err)
}

func bstreamUntilMarker(inFile, outFile string, marker []byte, startPos int64) int64 {
	in, err := os.Open(inFile)
	if err != nil {
		panic(err)
	}
	defer in.Close()

	out, err := os.OpenFile(outFile, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		panic(err)
	}
	defer out.Close()

	buffer := make([]byte, chunkSize)
	markerPos := startPos

	for {
		n, err := in.ReadAt(buffer, markerPos)
		if err != nil && err != io.EOF {
			panic(err)
		}

		if marker != nil {
			index := bytes.Index(buffer[:n], marker)
			if index >= 0 {
				out.Write(buffer[:index])
				return markerPos + int64(index)
			}
		}

		out.Write(buffer[:n])
		markerPos += int64(n)

		if err == io.EOF {
			break
		}
	}

	return markerPos
}

type Frame struct {
	Pos  int64
	Size int64
	Type string
}

func buildFrameTable(moviFile string) []Frame {
	file, err := os.Open(moviFile)
	if err != nil {
		panic(err)
	}
	defer file.Close()

	fileInfo, err := file.Stat()
	if err != nil {
		panic(err)
	}
	fileSize := fileInfo.Size()

	frameTable := []Frame{}
	buffer := make([]byte, chunkSize)

	iframeRegex := regexp.MustCompile("\x30\x31\x77\x62")
	bframeRegex := regexp.MustCompile("\x30\x30\x64\x63")

	for pos := int64(0); pos < fileSize; pos += chunkSize {
		n, err := file.ReadAt(buffer, pos)
		if err != nil && err != io.EOF {
			panic(err)
		}

		if audio {
			iframeMatches := iframeRegex.FindAllIndex(buffer[:n], -1)
			for _, match := range iframeMatches {
				frameTable = append(frameTable, Frame{Pos: pos + int64(match[0]), Type: "sound"})
			}
		}

		bframeMatches := bframeRegex.FindAllIndex(buffer[:n], -1)
		for _, match := range bframeMatches {
			frameTable = append(frameTable, Frame{Pos: pos + int64(match[0]), Type: "video"})
		}

		if err == io.EOF {
			break
		}
	}

	sort.Slice(frameTable, func(i, j int) bool {
		return frameTable[i].Pos < frameTable[j].Pos
	})

	for i := 0; i < len(frameTable); i++ {
		if i+1 < len(frameTable) {
			frameTable[i].Size = frameTable[i+1].Pos - frameTable[i].Pos
		} else {
			frameTable[i].Size = fileSize - frameTable[i].Pos
		}
	}

	return frameTable
}

func cleanFrames(frameTable []Frame) []Frame {
	clean := []Frame{}
	maxFrameSize := int64(0)

	for _, frame := range frameTable {
		if frame.Size > maxFrameSize {
			maxFrameSize = frame.Size
		}
	}

	if firstFrame {
		for _, frame := range frameTable {
			if frame.Type == "video" {
				clean = append(clean, frame)
				break
			}
		}
	}

	for _, frame := range frameTable {
		if float64(frame.Size) <= float64(maxFrameSize)*kill {
			clean = append(clean, frame)
		}
	}

	return clean
}

func applyEffect(clean []Frame) []Frame {
	var final []Frame

	switch mode {
	case "void":
		final = clean
	case "random":
		final = make([]Frame, len(clean))
		copy(final, clean)
		rand.Shuffle(len(final), func(i, j int) { final[i], final[j] = final[j], final[i] })
	case "reverse":
		final = make([]Frame, len(clean))
		for i, frame := range clean {
			final[len(clean)-1-i] = frame
		}
	case "invert":
		final = make([]Frame, len(clean))
		for i := 0; i < len(clean); i += 2 {
			if i+1 < len(clean) {
				final[i], final[i+1] = clean[i+1], clean[i]
			} else {
				final[i] = clean[i]
			}
		}
	case "bloom":
		repeat := countFrames
		frame := positFrames
		final = append(final, clean[:frame]...)
		for i := 0; i < repeat; i++ {
			final = append(final, clean[frame])
		}
		final = append(final, clean[frame+1:]...)
	case "pulse":
		pulselen := countFrames
		pulseryt := positFrames
		for i, frame := range clean {
			if i%pulselen == 0 {
				for j := 0; j < pulseryt; j++ {
					final = append(final, frame)
				}
			} else {
				final = append(final, frame)
			}
		}
	case "jiggle":
		amount := positFrames
		for i := range clean {
			j := constrain(i+int(rand.NormFloat64()*float64(amount)), 0, len(clean)-1)
			final = append(final, clean[j])
		}
	case "overlap":
		pulselen := countFrames
		pulseryt := positFrames
		for i := 0; i < len(clean); i += pulseryt {
			end := i + pulselen
			if end > len(clean) {
				end = len(clean)
			}
			final = append(final, clean[i:end]...)
		}
	default:
		fmt.Println("Unknown mode, using void")
		final = clean
	}

	return final
}

func writeFrames(moviFile, outFile string, frames []Frame) {
	in, err := os.Open(moviFile)
	if err != nil {
		panic(err)
	}
	defer in.Close()

	out, err := os.OpenFile(outFile, os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		panic(err)
	}
	defer out.Close()

	writer := bufio.NewWriter(out)
	writer.Write([]byte("movi"))

	buffer := make([]byte, 1024*1024) // 1MB buffer

	for _, frame := range frames {
		if frame.Pos != 0 && frame.Size != 0 {
			in.Seek(frame.Pos, 0)
			remaining := frame.Size
			for remaining > 0 {
				readSize := int64(len(buffer))
				if remaining < readSize {
					readSize = remaining
				}
				n, err := in.Read(buffer[:readSize])
				if err != nil && err != io.EOF {
					panic(err)
				}
				writer.Write(buffer[:n])
				remaining -= int64(n)
				if err == io.EOF {
					break
				}
			}
		}
	}

	writer.Flush()
}

func constrain(val, min, max int) int {
	if val < min {
		return min
	}
	if val > max {
		return max
	}
	return val
}