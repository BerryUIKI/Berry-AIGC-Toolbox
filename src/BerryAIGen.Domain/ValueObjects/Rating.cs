namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a rating value from 1 to 10 for image quality assessment.
/// </summary>
public sealed class Rating : ValueObject
{
    /// <summary>
    /// The minimum rating value.
    /// </summary>
    public const int Min = 1;

    /// <summary>
    /// The maximum rating value.
    /// </summary>
    public const int Max = 10;

    /// <summary>
    /// The default rating value for unrated images.
    /// </summary>
    public static readonly Rating Default = new(0);

    /// <summary>
    /// Gets the rating value.
    /// </summary>
    public int Value { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="Rating"/> class.
    /// </summary>
    /// <param name="value">The rating value.</param>
    private Rating(int value)
    {
        Value = value;
    }

    /// <summary>
    /// Creates a Rating from an integer value.
    /// </summary>
    /// <param name="value">The rating value (0-10).</param>
    /// <returns>A Rating instance.</returns>
    /// <exception cref="ArgumentOutOfRangeException">Thrown when the value is out of range.</exception>
    public static Rating From(int value)
    {
        if (value < 0 || value > Max)
            throw new ArgumentOutOfRangeException(nameof(value), $"Rating must be between 0 and {Max}.");

        return new Rating(value);
    }

    /// <summary>
    /// Gets whether this rating has been set (non-zero).
    /// </summary>
    public bool IsRated => Value > 0;

    /// <summary>
    /// Checks if this rating is within the valid range for actual ratings.
    /// </summary>
    public bool IsValidRating => Value >= Min && Value <= Max;

    /// <inheritdoc/>
    protected override IEnumerable<object?> GetEqualityComponents()
    {
        yield return Value;
    }

    /// <summary>
    /// Returns the string representation of the rating.
    /// </summary>
    public override string ToString() => Value == 0 ? "Unrated" : $"{Value}/{Max}";

    /// <summary>
    /// Implicitly converts Rating to int.
    /// </summary>
    public static implicit operator int(Rating rating) => rating.Value;

    /// <summary>
    /// Explicitly converts int to Rating.
    /// </summary>
    public static explicit operator Rating(int value) => From(value);

    /// <summary>
    /// Compares two Rating instances.
    /// </summary>
    public static bool operator <(Rating left, Rating right) => left.Value < right.Value;

    /// <summary>
    /// Compares two Rating instances.
    /// </summary>
    public static bool operator >(Rating left, Rating right) => left.Value > right.Value;

    /// <summary>
    /// Compares two Rating instances.
    /// </summary>
    public static bool operator <=(Rating left, Rating right) => left.Value <= right.Value;

    /// <summary>
    /// Compares two Rating instances.
    /// </summary>
    public static bool operator >=(Rating left, Rating right) => left.Value >= right.Value;
}