#!/usr/bin/python3
import time
timer_start = time.time_ns()

import argparse, os, re, random, struct
from itertools import repeat

print (" _                        _        ")
print ("| |                      | |       ")
print ("| |_ ___  _ __ ___   __ _| |_ ___  ")
print ("| __/ _ \| '_ ` _ \ / _` | __/ _ \ ")
print ("| || (_) | | | | | | (_| | || (_) |")
print (" \__\___/|_| |_| |_|\__,_|\__\___/ ")
print ("tomato.py v2.0 last update 21.03.2020")
print ("\\\\ Audio Video Interleave breaker")
print (" ")
print ("glitch tool made with love for the glitch art community <3")
print ("if you have any questions, would like to contact me")
print ("or even hire me for performance / research / education")
print ("you can shoot me an email at kaspar.ravel@gmail.com")
print ("___________________________________")
print (" ")
print ("wb. https://www.kaspar.wtf ")
print ("fb. https://www.facebook.com/kaspar.wtf ")
print ("ig. https://www.instagram.com/kaspar.wtf ")
print ("___________________________________")
print (" ")

#parse arguments
parser = argparse.ArgumentParser(add_help=True)
parser.add_argument("-i", "--input", help="input file")
parser.add_argument('-m', "--mode", action='store', dest='modevalue',help='choose mode - void, random, reverse, invert, bloom, pulse, jiggle, overlap', default='void')
parser.add_argument('-c', action='store', dest='countframes', default=1,help="how often to glitch (for modes that support it)")
parser.add_argument('-n', action='store', dest='positframes', default=1,help="how many frames in the glitch (for modes that support it)")
parser.add_argument('-a', action='store', dest='audio', default=0,help="attempt to preserve audio")
parser.add_argument('-ff', action='store', dest='firstframe', default=1,help="whether to keep first video frame")
parser.add_argument('-k', action='store', dest='kill', default=0.7, type=float,help="max framesize to kill while cleaning")

args = parser.parse_args()

filein = args.input
mode = args.modevalue
countframes = args.countframes
positframes = args.positframes
audio = args.audio
firstframe = args.firstframe
kill = args.kill

if filein is None or os.path.exists(filein) == False:
	print("> step 0/5: valid input file required!")
	print("use -h to see help")
	exit()
	

#define temp directory and files
temp_nb = random.randint(10000, 99999)
temp_dir = "temp-" + str(temp_nb)
temp_hdrl = temp_dir +"/hdrl.bin"
temp_movi = temp_dir +"/movi.bin"
temp_idx1 = temp_dir +"/idx1.bin"

os.mkdir(temp_dir)

#Define constrain function for jiggle :3
def constrain(val, min_val, max_val):
    return min(max_val, max(min_val, val))

######################################
### STREAM FILE INTO WORK DIR BINS ###
######################################

print("> step 1/5 : streaming into binary files")

def bstream_until_marker(bfilein, bfileout, marker=0, startpos=0):
	chunk = 1024
	filesize = os.path.getsize(bfilein)
	if marker :
		marker = str.encode(marker)

	with open(bfilein,'rb') as rd:
		with open(bfileout,'ab') as wr:
			for pos in range(startpos, filesize, chunk):
				rd.seek(pos)
				buffer = rd.read(chunk)

				if marker:
					if buffer.find(marker) > 0 :
						marker_pos = re.search(marker, buffer).start() # position is relative to buffer glitchedframes
						marker_pos = marker_pos + pos # position should be absolute now
						split = buffer.split(marker, 1)
						wr.write(split[0])
						return marker_pos
					else:
						wr.write(buffer)
				else:
					wr.write(buffer)

#make 3 files, 1 for each chunk
movi_marker_pos = bstream_until_marker(filein, temp_hdrl, "movi")
idx1_marker_pos = bstream_until_marker(filein, temp_movi, "idx1", movi_marker_pos)
bstream_until_marker(filein, temp_idx1, 0, idx1_marker_pos)

####################################
### FUN STUFF WITH VIDEO CONTENT ###
####################################

print("> step 2/5 : constructing frame index")

with open(temp_movi,'rb') as rd:
	chunk = 1024
	filesize = os.path.getsize(temp_movi)
	frame_table = []

	for pos in range(0, filesize, chunk):
		rd.seek(pos)
		buffer = rd.read(chunk)

        #build first list with all adresses 
		for m in (re.finditer(b'\x30\x31\x77\x62', buffer)): # find iframes
				if audio : frame_table.append([m.start() + pos, 'sound'])		
		for m in (re.finditer(b'\x30\x30\x64\x63', buffer)): # find b frames
			frame_table.append([m.start() + pos, 'video'])

		#then remember to sort the list
		frame_table.sort(key=lambda tup: tup[0])

	l = []
	l.append([0,0, 'void'])
	max_frame_size = 0

    #build tuples for each frame index with frame sizes
	for n in range(len(frame_table)):
		if n + 1 < len(frame_table):
			frame_size = frame_table[n + 1][0] - frame_table[n][0]
		else:
			frame_size = filesize - frame_table[n][0]
		max_frame_size = max(max_frame_size, frame_size)
		l.append([frame_table[n][0],frame_size, frame_table[n][1]])


########################
### TIME FOR SOME FX ###
########################

# variables that make shit work
clean = []
final = []

# keep first video frame or not
if firstframe :
	for x in l :
		if x[2] == 'video':
			clean.append(x)
			break

# clean the list by killing "big" frames
for x in l:
	if x[1] <= (max_frame_size * kill) :
		clean.append(x)

# FX modes

if mode == "void":
	print('> step 3/5 : mode void')
	final = clean

if mode == "random":
	print('> step 3/5 : mode random')
	final = random.sample(clean,len(clean))

if mode == "reverse":
	print('> step 3/5 : mode reverse')
	final = clean[::-1]

if mode == "invert":
	print('> step 3/5 : mode invert')
	final = sum(zip(clean[1::2], clean[::2]), ())

if mode == 'bloom':
	print('> step 3/5 : mode bloom')
	repeat = int(countframes)
	frame = int(positframes)
	## split list
	lista = clean[:frame]
	listb = clean[frame:]
	## rejoin list with bloom
	final = lista + ([clean[frame]]*repeat) + listb

if mode == 'pulse':
	print('> step 3/5 : mode pulse')
	pulselen = int(countframes)
	pulseryt = int(positframes)
	j = 0
	for x in clean:
		i = 0
		if(j % pulselen == 0):
			while i < pulselen :
				final.append(x)
				i = i + 1
		else:
			final.append(x)
			j = j + 1

if mode == "jiggle":
	print('> step 3/5 : mode jiggle')
	#print('*needs debugging lol help thx*') # didn't pandy's branch fix this?
	amount = int(positframes)
	final = [clean[constrain(x+int(random.gauss(0,amount)),0,len(clean)-1)] for x in range(0,len(clean))]

if mode == "overlap":
	print('> step 3/5 : mode overlap')
	pulselen = int(countframes)
	pulseryt = int(positframes)

	clean = [clean[i:i+pulselen] for i in range(0,len(clean),pulseryt)]
	final = [item for sublist in clean for item in sublist]

if mode == 'exponential':
	print('> step 3/5 : mode exponential')
	print('sorry, currently not implemented. using void..')	

if mode == 'swap':
	print('> step 3/5 : mode swap')
	print('sorry, currently not implemented. using void..')		


####################################
### PUT VIDEO FILE BACK TOGETHER ###
####################################

print("> step 4/5 : putting things back together")

#name new file
cname = '-c' + str(countframes) if int(countframes) > 1 else '' 
pname = '-n' + str(positframes) if int(positframes) > 1 else ''
fileout = filein[:-4] + '-' + mode + cname + pname + '.avi'

#delete old file
if os.path.exists(fileout):
	os.remove(fileout)

bstream_until_marker(temp_hdrl, fileout)

with open(temp_movi,'rb') as rd:
	filesize = os.path.getsize(temp_movi)
	with open(fileout,'ab') as wr:
		wr.write(struct.pack('<4s', b'movi'))
		for x in final:
			if x[0] != 0 and x[1] != 0:
				rd.seek(x[0])
				wr.write(rd.read(x[1]))

bstream_until_marker(temp_idx1, fileout)

#remove unnecessary temporary files and folders
os.remove(temp_hdrl)
os.remove(temp_movi)
os.remove(temp_idx1)
os.rmdir(temp_dir)

print("> step 5/5 : done - final idx size : " + str(len(final)))

timer_end = time.time_ns()
# time passed in ms
print("time elapsed: " + str(round((timer_end - timer_start) / 1000000, 2)) + "ms")