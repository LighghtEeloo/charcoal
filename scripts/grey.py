#!/usr/bin/env python3

help_header = \
"""\
grey - a command-line gre vocabulary assistant

Supports vim-style operations. Utilizes charcoal for word query.
To start, cd to the location of this program and run `echo "[]" > data/known.json`.
"""

help_cmds = \
"""\
Commands:
- <ENTER> : step forwards
- j/k [NUM] : step forwards / backwards by NUM
- g <NUM> : go to NUM
- c : mark current word as known
- h : show help messages
- w : save progress
- q/x : quit after save
"""

import json
import os
import time

args = {
    "bank_path": "data/bank.json",
    "known_path": "data/known.json",
}


class Traversor():
    def __init__(self, bank_path, known_path):
        known_old_path = known_path+".bak"
        with open(bank_path, 'r') as f:
            bank = json.load(f)
            self.bank = list(bank.keys())
        with open(known_path, 'r') as f:
            known = json.load(f)
            with open(known_old_path, 'w') as f:
                json.dump(known, f, indent=4)
            self.known = set(known)
        self.known_path = known_path
        self.i = 0

    def bound(self):
        i = self.i
        bank_len = len(self.bank)
        # keep i in bound
        i = i if i > 0 else i + bank_len * ((-i // bank_len) + 1)
        i = i % bank_len
        self.i = i

    def fix(self, step=1):
        self.bound()
        while self.bank[self.i] in self.known:
            self.i += step
            self.bound()

    def move_to(self, des):
        self.i = des
        self.fix()

    def move(self, step=1):
        self.i += step
        self.fix(1 if step > 0 else -1)

    def run_charcoal(self):
        os.system(f'charcoal query "{self.bank[self.i]}"')

    def run_clear(self):
        os.system("clear")

    def save(self):
        with open(self.known_path, 'w') as f:
            known = list(self.known)
            json.dump(known, f, indent=4)

    def interact(self):
        try:
            cmd = input()

            def num(default):
                nonlocal cmd
                s = cmd.split()
                if len(s) == 1:
                    return default
                return int(s[-1])

            if len(cmd) == 0:
                self.move(1)
            elif cmd.startswith("h") or cmd.startswith("?"):
                self.run_clear()
                print(help_header)
                print(help_cmds)
                input()
            elif cmd.startswith("j"):
                self.move(num(default=1))
            elif cmd.startswith("k"):
                self.move(-num(default=1))
            elif cmd.startswith("g"):
                self.move_to(num(self.i))
            elif cmd.startswith("c"):
                self.known.add(self.bank[self.i])
                self.move(1)
            elif cmd.startswith("w"):
                self.save()
                print("<<< (saving)")
                time.sleep(0.4)
            elif cmd.startswith("q") or cmd.startswith("x"):
                raise

        except:
            self.save()
            print("<<< (saved)")
            exit()

    def main(self):
        self.fix()
        while True:
            self.run_clear()
            print(f">>> [{self.i}]")
            self.run_charcoal()
            self.interact()


if __name__ == '__main__':
    tra = Traversor(**args)
    tra.main()
