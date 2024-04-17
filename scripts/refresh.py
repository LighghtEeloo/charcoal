import os

if __name__ == "__main__":
    home = os.path.expanduser("~")
    with open(os.path.join(home, ".zhistory"), 'r') as f:
        lines = [line for line in f.readlines() if line.startswith("ww ")]
    lines = [line.rstrip().lstrip("ww ") for line in lines]
    # for line in lines:
    #     for letter in line:
    #         # report if letter is not english
    #         if not letter.isalpha() and not letter.isspace() and not letter == '-':
    #             print(line)
    #             break
    for line in lines:
        os.system(f"charcoal query {line} -s --refresh")