namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a unique identifier for an Album entity.
/// </summary>
public sealed record AlbumId
{
    public Guid Value { get; }

    private AlbumId(Guid value)
    {
        Value = value;
    }

    public static AlbumId New() => new(Guid.NewGuid());
    public static AlbumId From(Guid value) => new(value);

    public override string ToString() => Value.ToString();

    public static implicit operator Guid(AlbumId id) => id.Value;
    public static explicit operator AlbumId(Guid value) => From(value);
}