using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeDependencyFulfilled
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeDependencyFulfilled); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeDependencyFulfilled)obj;
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize Fulfilled
            s.Write(value.Fulfilled);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeDependencyFulfilled)) as Rts.CnC.Messages.Client.TechTreeDependencyFulfilled;
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize Fulfilled
            s.Read(out value.Fulfilled);

            return value;
        }
        
    }
}
