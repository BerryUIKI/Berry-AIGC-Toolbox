using BerryAIGen.Domain.Common;

namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents the dimensions (width and height) of an image.
/// </summary>
public sealed class ImageDimensions : ValueObject
{
    /// <summary>
    /// Gets the width in pixels.
    /// </summary>
    public int Width { get; }

    /// <summary>
    /// Gets the height in pixels.
    /// </summary>
    public int Height { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="ImageDimensions"/> class.
    /// </summary>
    /// <param name="width">The width in pixels.</param>
    /// <param name="height">The height in pixels.</param>
    private ImageDimensions(int width, int height)
    {
        Width = width;
        Height = height;
    }

    /// <summary>
    /// Creates ImageDimensions from width and height values.
    /// </summary>
    /// <param name="width">The width in pixels.</param>
    /// <param name="height">The height in pixels.</param>
    /// <returns>An ImageDimensions instance.</returns>
    /// <exception cref="ArgumentOutOfRangeException">Thrown when dimensions are invalid.</exception>
    public static ImageDimensions From(int width, int height)
    {
        if (width <= 0)
            throw new ArgumentOutOfRangeException(nameof(width), "Width must be greater than 0.");

        if (height <= 0)
            throw new ArgumentOutOfRangeException(nameof(height), "Height must be greater than 0.");

        return new ImageDimensions(width, height);
    }

    /// <summary>
    /// Gets the aspect ratio (width / height).
    /// </summary>
    public double AspectRatio => (double)Width / Height;

    /// <summary>
    /// Gets the total pixel count (width × height).
    /// </summary>
    public long PixelCount => (long)Width * Height;

    /// <summary>
    /// Gets the megapixel count.
    /// </summary>
    public double Megapixels => PixelCount / 1_000_000.0;

    /// <summary>
    /// Checks if this is a landscape orientation image.
    /// </summary>
    public bool IsLandscape => Width > Height;

    /// <summary>
    /// Checks if this is a portrait orientation image.
    /// </summary>
    public bool IsPortrait => Height > Width;

    /// <summary>
    /// Checks if this is a square image.
    /// </summary>
    public bool IsSquare => Width == Height;

    /// <inheritdoc/>
    protected override IEnumerable<object?> GetEqualityComponents()
    {
        yield return Width;
        yield return Height;
    }

    /// <summary>
    /// Returns the string representation of the dimensions.
    /// </summary>
    public override string ToString() => $"{Width}×{Height}";

    /// <summary>
    /// Deconstructs the dimensions into width and height.
    /// </summary>
    public void Deconstruct(out int width, out int height)
    {
        width = Width;
        height = Height;
    }
}