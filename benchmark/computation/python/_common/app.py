import sys


def calc_pi(iterations):
    pi = 0.0
    denominator = 1.0
    sum = 0.0
    custom_number = 0.0
    for x in range(iterations):
        if x % 2 == 0:
            pi = pi + (1 / denominator)
        else:
            pi = pi - (1 / denominator)
        denominator = denominator + 2

        # custom
        sum += pi
        mod_3 = x % 3
        if mod_3 == 0:
            custom_number += pi
        elif mod_3 == 1:
            custom_number -= pi
        else:
            custom_number /= 2
    pi = pi * 4
    return [pi, sum, custom_number]


def main():
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        line = line.strip()
        if not line:
            continue
        iterations = int(line)
        [pi, sum, custom_number] = calc_pi(iterations)
        print(f"{pi};{sum};{custom_number}", flush=True)


if __name__ == "__main__":
    main()
