import sys

if __name__ == "__main__":
    args = sys.argv

    x = f"{int(float(args[1])):#0{args[2]}x}"[2:]

    print(x)
