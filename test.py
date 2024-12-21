import os, pty, serial, time
from tkinter import Tk, Canvas
import threading

DIM = 20000

# Tkinter
root = Tk()
canvas = Canvas(root, width=DIM, height=DIM)

# Global state
on: bool = False

master, slave = pty.openpty()
s_name = os.ttyname(slave)

print(s_name)

ser = serial.Serial(s_name)

# To Write to the device
ser.write(b'Your text')


def _from_rgb(rgb):
    """translates an rgb tuple of int to a tkinter friendly color code
    """
    return "#%02x%02x%02x" % rgb


def serial():
    time.sleep(2)

    # To read from the device
    while True:
        got = os.read(master, 10)
        print(got)
        # print(' '.join(format(ord(x), 'b') for x in got))
        rgb_val = 255 if got == b'B' else 0
        canvas.itemconfig(lamp_id, fill=_from_rgb((rgb_val, rgb_val, rgb_val)))


if __name__ == '__main__':
    global lamp_id

    server_thread = threading.Thread(target=serial)
    server_thread.start()

    root.configure(background='black')
    lamp_id = canvas.create_rectangle(0, 0, DIM, DIM, fill="white", outline='black')

    canvas.pack()
    root.mainloop()

    server_thread.join()
