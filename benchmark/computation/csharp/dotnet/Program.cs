using System.Globalization;

while (Console.ReadLine() is { } line)
{
	line = line.Trim();
	if (line.Length == 0)
	{
		continue;
	}

	var (pi, sum, customNumber) = CalculatePi(int.Parse(line, CultureInfo.InvariantCulture));
	Console.WriteLine(string.Create(CultureInfo.InvariantCulture, $"{pi};{sum};{customNumber}"));
}

static (double Pi, double Sum, double CustomNumber) CalculatePi(int iterations)
{
	double pi = 0.0d;
	double denominator = 1.0d;
	double sum = 0.0d;
	double customNumber = 0.0d;

	for (int x = 0; x < iterations; x++)
	{
		if (x % 2 == 0)
		{
			pi += (1d / denominator);
		}
		else
		{
			pi -= (1d / denominator);
		}
		denominator += 2d;

		// custom
		sum += pi;
		switch (x % 3)
		{
			case 0:
				customNumber += pi;
				break;
			case 1:
				customNumber -= pi;
				break;
			case 2:
				customNumber /= 2d;
				break;
		}
	}
	pi *= 4d;
	return (pi, sum, customNumber);
}
