using System.Buffers;
using System.Globalization;
using System.Net;
using System.Text.Unicode;

const int port = 3000;
await Run(port);
return;

static async Task Run(int port)
{
	var listener = new HttpListener();
	listener.Prefixes.Add($"http://*:{port}/");
	listener.Start();
	Console.WriteLine($"Running on port {port}");

	var buffer = new byte[256];

	while (true)
	{
		var context = await listener.GetContextAsync().ConfigureAwait(false);
		var request = context.Request;
		using var response = context.Response;

		var iterationsParam = request.QueryString["iterations"]!;
		var iterations = int.Parse(iterationsParam);
		var responseLength = CreateResponse(buffer, iterations);

		response.ContentLength64 = responseLength;
		await using var output = response.OutputStream;
		await output.WriteAsync(buffer.AsMemory(0, responseLength));
	}

	// ReSharper disable once FunctionNeverReturns
}

static int CreateResponse(Span<byte> buffer, int iterations, bool addHeader = false)
{
#if NET8_0_OR_GREATER
	const byte semicolon = (byte)';';
	var span = buffer;
#else
	const char semicolon = ';';
	var pool = ArrayPool<char>.Shared.Rent(256);
	var span = pool.AsSpan();
#endif

	var (pi, sum, customNumber) = CalculatePi(iterations);

	var index = 0;

	if (addHeader)
	{
#if NET8_0_OR_GREATER
		var header = "HTTP/1.1 200 OK\r\n\r\n"u8;
#else
		const string header = "HTTP/1.1 200 OK\r\n\r\n";
#endif
		header.CopyTo(span);
		index += header.Length;
	}

	pi.TryFormat(span[index..], out var piWritten, provider: CultureInfo.InvariantCulture);
	index += piWritten;
	span[index] = semicolon;
	index++;
	sum.TryFormat(span[index..], out var sumWritten, provider: CultureInfo.InvariantCulture);
	index += sumWritten;
	span[index] = semicolon;
	index++;
	customNumber.TryFormat(span[index..], out var customNumberWritten, provider: CultureInfo.InvariantCulture);
	index += customNumberWritten;

#if NET8_0_OR_GREATER
	return index;
#else
	Utf8.FromUtf16(span[..index], buffer, out _, out var bufferLength);
	ArrayPool<char>.Shared.Return(pool);
	return bufferLength;
#endif
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
