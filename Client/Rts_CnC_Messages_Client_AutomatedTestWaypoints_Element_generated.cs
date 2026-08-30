using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AutomatedTestWaypoints_Element
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element)obj;
            //  Serialize Name
            s.Write(value.Name);
            //  Serialize Position
            s.Write(value.Position);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element value = default(Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element);
            DeserializeValue(s, ref value);
            return value;
        }
        
        public static void DeserializeValue(System.IO.Stream s, ref Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element value)
        {
            var valueRef = __makeref(value);
            //  Deserialize Name
            s.Read(out value.Name);
            //  Deserialize Position
            s.Read(out value.Position);

        }
    }
}
