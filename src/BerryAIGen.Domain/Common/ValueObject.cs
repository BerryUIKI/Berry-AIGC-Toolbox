namespace BerryAIGen.Domain.Common;

/// <summary>
/// Base class for value objects in Domain-Driven Design.
/// Value objects are immutable objects that are defined by their attributes, not by a unique identity.
/// Two value objects are equal if all their attributes are equal.
/// </summary>
public abstract class ValueObject : IEquatable<ValueObject>
{
    /// <summary>
    /// Gets the atomic values that define this value object's equality.
    /// </summary>
    /// <returns>An enumerable of atomic values used for equality comparison.</returns>
    protected abstract IEnumerable<object?> GetEqualityComponents();

    /// <summary>
    /// Determines whether the specified value object is equal to the current value object.
    /// </summary>
    /// <param name="other">The value object to compare with the current value object.</param>
    /// <returns>true if the specified value object is equal to the current value object; otherwise, false.</returns>
    public bool Equals(ValueObject? other)
    {
        if (other is null)
        {
            return false;
        }

        if (other.GetType() != GetType())
        {
            return false;
        }

        return GetEqualityComponents().SequenceEqual(other.GetEqualityComponents());
    }

    /// <summary>
    /// Determines whether the specified object is equal to the current value object.
    /// </summary>
    /// <param name="obj">The object to compare with the current value object.</param>
    /// <returns>true if the specified object is equal to the current value object; otherwise, false.</returns>
    public override bool Equals(object? obj)
    {
        return Equals(obj as ValueObject);
    }

    /// <summary>
    /// Returns a hash code for the current value object based on its atomic values.
    /// </summary>
    /// <returns>A hash code for the current value object.</returns>
    public override int GetHashCode()
    {
        return GetEqualityComponents()
            .Select(x => x?.GetHashCode() ?? 0)
            .Aggregate((x, y) => x ^ y);
    }

    /// <summary>
    /// Determines whether two value objects are equal.
    /// </summary>
    /// <param name="left">The first value object to compare.</param>
    /// <param name="right">The second value object to compare.</param>
    /// <returns>true if the value objects are equal; otherwise, false.</returns>
    public static bool operator ==(ValueObject? left, ValueObject? right)
    {
        return Equals(left, right);
    }

    /// <summary>
    /// Determines whether two value objects are not equal.
    /// </summary>
    /// <param name="left">The first value object to compare.</param>
    /// <param name="right">The second value object to compare.</param>
    /// <returns>true if the value objects are not equal; otherwise, false.</returns>
    public static bool operator !=(ValueObject? left, ValueObject? right)
    {
        return !Equals(left, right);
    }
}