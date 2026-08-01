namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a unique identifier for an Image entity.
/// This is a strongly-typed ID to prevent primitive obsession and provide type safety.
/// </summary>
public sealed record ImageId
{
    /// <summary>
    /// Gets the value of the image identifier.
    /// </summary>
    public Guid Value { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="ImageId"/> class.
    /// </summary>
    /// <param name="value">The unique identifier value.</param>
    private ImageId(Guid value)
    {
        Value = value;
    }

    /// <summary>
    /// Creates a new unique ImageId.
    /// </summary>
    /// <returns>A new ImageId with a generated GUID.</returns>
    public static ImageId New() => new(Guid.NewGuid());

    /// <summary>
    /// Creates an ImageId from an existing GUID value.
    /// </summary>
    /// <param name="value">The GUID value.</param>
    /// <returns>An ImageId instance.</returns>
    public static ImageId From(Guid value) => new(value);

    /// <summary>
    /// Converts the ImageId to its string representation.
    /// </summary>
    /// <returns>The string representation of the ImageId.</returns>
    public override string ToString() => Value.ToString();

    /// <summary>
    /// Implicitly converts ImageId to Guid.
    /// </summary>
    public static implicit operator Guid(ImageId id) => id.Value;

    /// <summary>
    /// Explicitly converts Guid to ImageId.
    /// </summary>
    public static explicit operator ImageId(Guid value) => From(value);
}