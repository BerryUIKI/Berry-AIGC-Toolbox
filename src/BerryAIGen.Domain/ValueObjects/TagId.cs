namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a unique identifier for a Tag entity.
/// </summary>
public sealed record TagId
{
    public Guid Value { get; }

    private TagId(Guid value)
    {
        Value = value;
    }

    public static TagId New() => new(Guid.NewGuid());
    public static TagId From(Guid value) => new(value);

    public override string ToString() => Value.ToString();

    public static implicit operator Guid(TagId id) => id.Value;
    public static explicit operator TagId(Guid value) => From(value);
}