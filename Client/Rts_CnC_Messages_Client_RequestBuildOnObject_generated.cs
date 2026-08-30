using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestBuildOnObject
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestBuildOnObject); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestBuildOnObject)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize ExistingObjectPlayerId
            s.Write(value.ExistingObjectPlayerId);
            //  Serialize ExistingObjectId
            s.Write(value.ExistingObjectId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Orientation
            s.Write(value.Orientation);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestBuildOnObject)) as Rts.CnC.Messages.Client.RequestBuildOnObject;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize ExistingObjectPlayerId
            s.Read(out value.ExistingObjectPlayerId);
            //  Deserialize ExistingObjectId
            s.Read(out value.ExistingObjectId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Orientation
            s.Read(out value.Orientation);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
